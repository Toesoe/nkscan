typedef struct sg_io_hdr
{
    int interface_id;           /* [i] 'S' (required) */
    int dxfer_direction;        /* [i] */
    unsigned char cmd_len;      /* [i] */
    unsigned char mx_sb_len;    /* [i] */
    unsigned short iovec_count; /* [i] */
    unsigned int dxfer_len;     /* [i] */
    void * dxferp;              /* [i], [*io] */
    unsigned char * cmdp;       /* [i], [*i]  */
    unsigned char * sbp;        /* [i], [*o]  */
    unsigned int timeout;       /* [i] unit: millisecs */
    unsigned int flags;         /* [i] */
    int pack_id;                /* [i->o] */
    void * usr_ptr;             /* [i->o] */
    unsigned char status;       /* [o] */
    unsigned char masked_status;/* [o] */
    unsigned char msg_status;   /* [o] */
    unsigned char sb_len_wr;    /* [o] */
    unsigned short host_status; /* [o] */
    unsigned short driver_status;/* [o] */
    int resid;                  /* [o] */
    unsigned int duration;      /* [o] */
    unsigned int info;          /* [o] */
} sg_io_hdr_t;  /* 64 bytes long (on i386) */
